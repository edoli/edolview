//! Compact caption hit testing; Windows owns every system window action.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, Weak,
};

use color_eyre::eyre::{eyre, Result};
use eframe::egui::{self, Rect};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::ScreenToClient,
    UI::{
        HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
        Input::KeyboardAndMouse::{
            GetCapture, ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT,
        },
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

const STATE_ID: &str = "native_window_chrome";
const REGIONS_ID: &str = "native_window_regions";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum Button {
    Close = 1,
    Maximize,
    Minimize,
}

impl Button {
    const ALL: [Self; 3] = [Self::Close, Self::Maximize, Self::Minimize];
    fn hit(self) -> u32 {
        match self {
            Self::Close => HTCLOSE,
            Self::Maximize => HTMAXBUTTON,
            Self::Minimize => HTMINBUTTON,
        }
    }
    fn from_hit(hit: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|button| button.hit() == hit)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Regions {
    pub title: Rect,
    pub buttons: [Rect; 3],
    pub layer: egui::LayerId,
}

#[derive(Default)]
struct Geometry {
    regions: Option<Regions>,
    exclusions: Vec<Rect>,
    pixels_per_point: f32,
    fullscreen: bool,
}

#[derive(Default)]
struct State {
    geometry: Geometry,
    hovered: Option<Button>,
    pressed: Option<Button>,
    errors: Vec<String>,
}

struct Shared {
    hwnd: usize,
    ctx: egui::Context,
    state: Mutex<State>,
    attached: AtomicBool,
}

pub(crate) struct NativeChrome {
    // HWND makes this owner !Send; installation and removal stay on the UI thread.
    hwnd: HWND,
    shared: Arc<Shared>,
}

impl NativeChrome {
    pub(crate) fn attach(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let RawWindowHandle::Win32(handle) = cc.window_handle()?.as_raw() else {
            return Err(eyre!("Expected a Windows window handle"));
        };
        let hwnd = handle.hwnd.get() as HWND;
        let shared = Arc::new(Shared {
            hwnd: hwnd as usize,
            ctx: cc.egui_ctx.clone(),
            state: Mutex::new(State::default()),
            attached: AtomicBool::new(true),
        });
        let reference = Arc::into_raw(shared.clone()) as usize;
        // The subclass owns one Arc until removal/WM_NCDESTROY. HWND comes from
        // eframe on this UI thread; callback-local clones protect native reentry.
        if unsafe { SetWindowSubclass(hwnd, Some(window_proc), reference, reference) } == 0 {
            unsafe { drop(Arc::from_raw(reference as *const Shared)) };
            return Err(native_error("install Windows caption handling"));
        }
        let chrome = Self { hwnd, shared };
        // Keep native caption/system-menu styles for WM_SYSCOMMAND. Only the
        // visible frame changes; Windows still chooses sizes, positions and animations.
        if !refresh_frame(hwnd) {
            return Err(native_error("update Windows frame"));
        }
        cc.egui_ctx
            .data_mut(|data| data.insert_temp(egui::Id::unique(STATE_ID), Arc::downgrade(&chrome.shared)));
        Ok(chrome)
    }

    pub(crate) fn update(&self, ctx: &egui::Context) -> Vec<String> {
        let regions = ctx.data(|data| data.get_temp::<Regions>(egui::Id::unique(REGIONS_ID)));
        let exclusions = regions.map_or_else(Vec::new, |regions| {
            ctx.memory(|memory| {
                memory
                    .areas()
                    .visible_layer_ids()
                    .into_iter()
                    .filter(|layer| layer.order > regions.layer.order && layer.order != egui::Order::Tooltip)
                    .filter_map(|layer| memory.area_rect(layer.id))
                    .collect()
            })
        });
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        state.geometry = Geometry {
            regions,
            exclusions,
            pixels_per_point: ctx.pixels_per_point(),
            fullscreen: ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
        };
        let errors = std::mem::take(&mut state.errors);
        drop(state);
        errors
    }
}

impl Drop for NativeChrome {
    fn drop(&mut self) {
        let reference = Arc::as_ptr(&self.shared);
        // UI-thread-only removal cannot race the callback. WM_NCDESTROY may
        // already have detached and released the subclass's Arc.
        if self.shared.attached.load(Ordering::Relaxed)
            && unsafe { RemoveWindowSubclass(self.hwnd, Some(window_proc), reference as usize) } != 0
        {
            self.shared.attached.store(false, Ordering::Relaxed);
            unsafe { drop(Arc::from_raw(reference)) };
        }
    }
}

pub(crate) fn store_regions(ctx: &egui::Context, regions: Regions) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique(REGIONS_ID), regions));
}
fn shared(ctx: &egui::Context) -> Option<Arc<Shared>> {
    ctx.data(|data| data.get_temp::<Weak<Shared>>(egui::Id::unique(STATE_ID)))
        .and_then(|state| state.upgrade())
}
pub(super) fn button_state(ctx: &egui::Context, button: Button) -> (bool, bool) {
    shared(ctx).map_or((false, false), |shared| {
        let state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let hovered = state.hovered == Some(button);
        (hovered, hovered && state.pressed == Some(button))
    })
}
pub(super) fn is_maximized(ctx: &egui::Context) -> bool {
    shared(ctx).is_some_and(|state| unsafe { IsZoomed(state.hwnd as HWND) } != 0)
}
pub(super) fn activate(ctx: &egui::Context, button: Button) {
    if let Some(shared) = shared(ctx) {
        post_button_command(&shared, button);
    }
}

fn system_command(button: Button, maximized: bool) -> u32 {
    match button {
        Button::Close => SC_CLOSE,
        Button::Minimize => SC_MINIMIZE,
        Button::Maximize => {
            if maximized {
                SC_RESTORE
            } else {
                SC_MAXIMIZE
            }
        }
    }
}

fn post_button_command(shared: &Shared, button: Button) {
    let hwnd = shared.hwnd as HWND;
    let command = system_command(button, unsafe { IsZoomed(hwnd) } != 0);
    // Mouse, keyboard and accessibility all use the native system-command path.
    // No ShowWindow/SetWindowPos implementation of minimize/maximize/restore.
    if unsafe { PostMessageW(hwnd, WM_SYSCOMMAND, command as WPARAM, 0) } == 0 {
        report(shared, "execute window system command");
    }
}

fn take_release(state: &mut State, hit: Option<Button>) -> Option<Button> {
    let pressed = state.pressed.take();
    state.hovered = hit;
    pressed.filter(|button| Some(*button) == hit)
}

fn release_capture(hwnd: HWND) {
    if unsafe { GetCapture() } == hwnd {
        unsafe { ReleaseCapture() };
    }
}

fn cancel_press(shared: &Shared) {
    let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
    state.pressed = None;
    state.hovered = None;
    drop(state);
    shared.ctx.request_repaint();
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    reference: usize,
) -> LRESULT {
    let pointer = reference as *const Shared;
    // The installed subclass owns this pointer. A local Arc survives reentrant
    // native calls, including WM_NCDESTROY; never hold our mutex across Win32 calls.
    let shared = unsafe {
        Arc::increment_strong_count(pointer);
        Arc::from_raw(pointer)
    };
    match message {
        WM_NCCALCSIZE if wparam != 0 && has_resize_frame(hwnd) => {
            // Windows provides NCCALCSIZE_PARAMS. Keep the native side/bottom
            // frame and extend only the top, exactly as the GPUI backend does.
            let params = lparam as *mut NCCALCSIZE_PARAMS;
            let top = unsafe { (*params).rgrc[0].top };
            let result = unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
            let inset = if unsafe { IsZoomed(hwnd) } != 0 {
                frame_size(hwnd).1
            } else {
                0
            };
            unsafe { (*params).rgrc[0].top = top + inset };
            return result;
        }
        WM_NCHITTEST => {
            let default = unsafe { DefWindowProcW(hwnd, message, wparam, lparam) } as u32;
            let mut point = message_point(lparam);
            let mut client = RECT::default();
            if unsafe { ScreenToClient(hwnd, &mut point) != 0 && GetClientRect(hwnd, &mut client) != 0 } {
                let frame = if has_resize_frame(hwnd) && unsafe { IsZoomed(hwnd) } == 0 {
                    Some(frame_size(hwnd))
                } else {
                    None
                };
                let state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                return hit_test(&state.geometry, point, client.right, frame, default) as LRESULT;
            }
            return default as LRESULT;
        }
        WM_MOUSEACTIVATE if Button::from_hit(lparam as u16 as u32).is_some() => return MA_ACTIVATE as LRESULT,
        WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK if Button::from_hit(wparam as u32).is_some() => {
            let button = Button::from_hit(wparam as u32).unwrap();
            {
                let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                state.pressed = Some(button);
                state.hovered = Some(button);
            }
            shared.ctx.request_repaint();
            // Only input tracking is ours. Capture supplies a release outside the
            // button; clear state before releasing capture to survive native reentry.
            unsafe { SetCapture(hwnd) };
            if unsafe { GetCapture() } != hwnd {
                cancel_press(&shared);
            }
            return 0;
        }
        WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
            let pressed = shared.state.lock().unwrap_or_else(|e| e.into_inner()).pressed.is_some();
            if pressed || message == WM_NCMOUSEMOVE {
                let hit = if message == WM_NCMOUSEMOVE {
                    Button::from_hit(wparam as u32)
                } else {
                    button_at(&shared, message_point(lparam))
                };
                shared.state.lock().unwrap_or_else(|e| e.into_inner()).hovered = hit;
                shared.ctx.request_repaint();
                if message == WM_NCMOUSEMOVE {
                    let mut tracking = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE | TME_NONCLIENT,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    unsafe { TrackMouseEvent(&mut tracking) };
                }
                if pressed || hit.is_some() {
                    return 0;
                }
            }
        }
        WM_LBUTTONUP | WM_NCLBUTTONUP => {
            let mut point = message_point(lparam);
            if message == WM_NCLBUTTONUP {
                unsafe { ScreenToClient(hwnd, &mut point) };
            }
            let hit = button_at(&shared, point);
            let (pressed, clicked) = {
                let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                (state.pressed.is_some(), take_release(&mut state, hit))
            };
            if pressed {
                release_capture(hwnd);
                shared.ctx.request_repaint();
                if let Some(button) = clicked {
                    post_button_command(&shared, button);
                }
                return 0;
            }
            if message == WM_NCLBUTTONUP && Button::from_hit(wparam as u32).is_some() {
                return 0;
            }
        }
        WM_NCMOUSELEAVE => {
            let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.pressed.is_none() {
                state.hovered = None;
            }
            drop(state);
            shared.ctx.request_repaint();
        }
        WM_NCMOUSEHOVER if Button::from_hit(wparam as u32).is_some() => return 0,
        WM_CAPTURECHANGED => cancel_press(&shared),
        WM_CANCELMODE => {
            cancel_press(&shared);
            release_capture(hwnd);
        }
        WM_ACTIVATE if wparam as u16 as u32 == WA_INACTIVE => {
            cancel_press(&shared);
            release_capture(hwnd);
        }
        WM_NCDESTROY => {
            unsafe { RemoveWindowSubclass(hwnd, Some(window_proc), id) };
            if shared.attached.swap(false, Ordering::Relaxed) {
                unsafe { drop(Arc::from_raw(pointer)) };
            }
        }
        _ => {}
    }
    // Standard title dragging, borders, system commands and Alt+F4 remain native.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

fn button_at(shared: &Shared, point: POINT) -> Option<Button> {
    let state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
    Button::from_hit(caption_hit(&state.geometry, point))
}

fn report(shared: &Shared, operation: &str) {
    let error = native_error(operation).to_string();
    shared.state.lock().unwrap_or_else(|e| e.into_inner()).errors.push(error);
    shared.ctx.request_repaint();
}
fn native_error(operation: &str) -> color_eyre::eyre::Report {
    eyre!("Could not {operation}: {}", std::io::Error::last_os_error())
}
fn refresh_frame(hwnd: HWND) -> bool {
    unsafe {
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
        ) != 0
    }
}
fn has_resize_frame(hwnd: HWND) -> bool {
    unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 & WS_THICKFRAME != 0 }
}
fn frame_size(hwnd: HWND) -> (i32, i32) {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let padding = unsafe { GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi) };
    unsafe {
        (
            GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi) + padding,
            GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi) + padding,
        )
    }
}
fn top_resize_hit(point: POINT, width: i32, x: i32, y: i32) -> Option<u32> {
    if point.y < 0 || point.y >= y || point.x < -x || point.x >= width + x {
        return None;
    }
    Some(if point.x < x {
        HTTOPLEFT
    } else if point.x >= width - x {
        HTTOPRIGHT
    } else {
        HTTOP
    })
}
fn message_point(lparam: LPARAM) -> POINT {
    POINT {
        x: lparam as u16 as i16 as i32,
        y: (lparam >> 16) as u16 as i16 as i32,
    }
}
fn hit_test(geometry: &Geometry, point: POINT, width: i32, frame: Option<(i32, i32)>, default: u32) -> u32 {
    let role = caption_hit(geometry, point);
    // Published UI rectangles are client-relative in both window states. Never
    // use DWM's fixed system button rectangles for compact buttons.
    if Button::from_hit(role).is_some() || geometry.fullscreen {
        return role;
    }
    if let Some((x, y)) = frame {
        if let Some(hit) = top_resize_hit(point, width, x, y) {
            return hit;
        }
        if matches!(
            default,
            HTLEFT | HTRIGHT | HTBOTTOM | HTBOTTOMLEFT | HTBOTTOMRIGHT | HTTOP | HTTOPLEFT | HTTOPRIGHT
        ) {
            return default;
        }
    }
    role
}

fn caption_hit(geometry: &Geometry, point: POINT) -> u32 {
    // GPUI does not expose non-client title-bar controls in fullscreen.
    if geometry.fullscreen {
        return HTCLIENT;
    }
    let point = egui::pos2(
        point.x as f32 / geometry.pixels_per_point,
        point.y as f32 / geometry.pixels_per_point,
    );
    let inside = |rect: Rect| {
        point.x >= rect.left() && point.x < rect.right() && point.y >= rect.top() && point.y < rect.bottom()
    };
    if geometry.exclusions.iter().any(|rect| inside(*rect)) {
        return HTCLIENT;
    }
    if let Some(regions) = geometry.regions {
        for (rect, button) in regions.buttons.into_iter().zip(Button::ALL) {
            if inside(rect) {
                return button.hit();
            }
        }
        if inside(regions.title) {
            return HTCAPTION;
        }
    }
    HTCLIENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_regions_and_popups_preserve_native_roles() {
        for scale in [1.0, 1.5, 2.0] {
            let rect = |x, w| Rect::from_min_size(egui::pos2(x, 0.0), egui::vec2(w, 32.0));
            let mut geometry = Geometry {
                pixels_per_point: scale,
                regions: Some(Regions {
                    title: rect(0.0, 500.0),
                    buttons: [rect(564.0, 32.0), rect(532.0, 32.0), rect(500.0, 32.0)],
                    layer: egui::LayerId::background(),
                }),
                ..Default::default()
            };
            for (x, expected) in [
                (100.0, HTCAPTION),
                (523.0, HTMINBUTTON),
                (569.0, HTCLOSE),
                (615.0, HTCLIENT),
                (638.0, HTCLIENT),
            ] {
                assert_eq!(
                    caption_hit(
                        &geometry,
                        POINT {
                            x: (x * scale) as i32,
                            y: (16.0 * scale) as i32
                        }
                    ),
                    expected
                );
            }
            geometry.exclusions.push(rect(500.0, 138.0));
            assert_eq!(
                caption_hit(
                    &geometry,
                    POINT {
                        x: (569.0 * scale) as i32,
                        y: (16.0 * scale) as i32
                    }
                ),
                HTCLIENT
            );
        }
    }

    #[test]
    fn compact_buttons_work_in_both_window_states_and_commands_remain_native() {
        for scale in [1.0, 1.5, 2.0] {
            let rect = |x| Rect::from_min_size(egui::pos2(x, 0.0), egui::vec2(32.0, 32.0));
            let geometry = Geometry {
                pixels_per_point: scale,
                regions: Some(Regions {
                    title: Rect::NOTHING,
                    buttons: [rect(688.0), rect(656.0), rect(624.0)],
                    layer: egui::LayerId::background(),
                }),
                ..Default::default()
            };
            for maximized in [false, true] {
                let frame = if maximized { None } else { Some((11, 11)) };
                for (button, rect) in Button::ALL.into_iter().zip(geometry.regions.unwrap().buttons) {
                    for x in [rect.left() + 1.0, rect.center().x, rect.right() - 1.0] {
                        for y in [1.0, 16.0, 31.0] {
                            let point = POINT {
                                x: (x * scale) as i32,
                                y: (y * scale) as i32,
                            };
                            assert_eq!(
                                hit_test(&geometry, point, (720.0 * scale) as i32, frame, HTCLIENT),
                                button.hit()
                            );
                        }
                    }
                }
                assert_eq!(
                    system_command(Button::Maximize, maximized),
                    if maximized { SC_RESTORE } else { SC_MAXIMIZE }
                );
                assert_eq!(system_command(Button::Minimize, maximized), SC_MINIMIZE);
                assert_eq!(system_command(Button::Close, maximized), SC_CLOSE);
            }
        }
    }

    #[test]
    fn release_or_cancellation_never_activates_twice_or_the_wrong_button() {
        for button in Button::ALL {
            let mut state = State {
                pressed: Some(button),
                ..Default::default()
            };
            assert_eq!(take_release(&mut state, Some(button)), Some(button));
            assert_eq!(take_release(&mut state, Some(button)), None);
            state.pressed = Some(button);
            assert_eq!(take_release(&mut state, None), None);
            state.pressed = Some(button);
            let other = Button::ALL.into_iter().find(|other| *other != button).unwrap();
            assert_eq!(take_release(&mut state, Some(other)), None);
            state.pressed = None;
            assert_eq!(take_release(&mut state, Some(button)), None);
        }
    }

    #[test]
    fn top_border_and_signed_coordinates_have_no_corner_gaps() {
        for frame in [8, 11, 16] {
            for x in -frame..720 + frame {
                for y in 0..frame {
                    assert!(top_resize_hit(POINT { x, y }, 720, frame, frame).is_some());
                }
            }
            assert_eq!(top_resize_hit(POINT { x: 100, y: frame }, 720, frame, frame), None);
        }
        let packed = ((-200_i16 as u16 as usize) << 16) | (-1920_i16 as u16 as usize);
        let point = message_point(packed as LPARAM);
        assert_eq!((point.x, point.y), (-1920, -200));
    }
}
