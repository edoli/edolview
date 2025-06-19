package kr.edoli.edolview;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Window;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.system.*;

public final class Windows {
    // DWM attribute constants
    public static final int DWMWA_BORDER_COLOR = 34;
    public static final int DWMWA_CAPTION_COLOR = 35;
    public static final int DWMWA_TEXT_COLOR = 36;

    // Load the dwmapi.dll library and cache the function address
    private static final SharedLibrary DWMAPI = Library.loadNative("dwmapi", "dwmapi");
    private static final long DwmSetWindowAttribute = DWMAPI.getFunctionAddress("DwmSetWindowAttribute");

    private Windows() {}

    /**
     * Convert RGB to COLORREF (0x00BBGGRR)
     */
    private static int toCOLORREF(int r, int g, int b) {
        return ((b & 0xFF) << 16) | ((g & 0xFF) << 8) | (r & 0xFF);
    }

    /**
     * Set a DWM window attribute using LWJGL's JNI FFI
     *
     * @param hwndAddr    The native HWND address
     * @param attribute   One of the DWMWA_* constants
     * @param colorRef    COLORREF value
     */
    public static void setColor(long hwndAddr, int attribute, int colorRef) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            // Allocate an off-heap buffer for the COLORREF
            var buf = stack.mallocInt(1);
            buf.put(0, colorRef);
            long ptr = MemoryUtil.memAddress(buf);

            // HRESULT DwmSetWindowAttribute(HWND hwnd, DWORD attr, LPCVOID pvAttr, DWORD cbAttr)
            int hr = JNI.callPPI(
                    hwndAddr,  // HWND handle
                    attribute,  // attribute constant
                    ptr,  // pointer to COLORREF
                    Integer.BYTES,  // size of DWORD
                    DwmSetWindowAttribute // function address
            );
            if (hr != 0) {
                throw new RuntimeException("DwmSetWindowAttribute failed, HRESULT: " + hr);
            }
        }
    }

    public static void winit(Lwjgl3Window window) {
        long hwnd = GLFWNativeWin32.glfwGetWin32Window(window.getWindowHandle());
        if (hwnd == 0L) {
            System.err.println("Failed to get foreground HWND.");
            return;
        }

        try {
            setColor(hwnd, DWMWA_BORDER_COLOR, toCOLORREF(45, 45, 45));
            setColor(hwnd, DWMWA_CAPTION_COLOR, toCOLORREF(24, 24, 24));
            // setColor(hwnd, DWMWA_TEXT_COLOR, redColor);
        } catch (Exception e) {
            e.printStackTrace();
            System.err.println("Window color update failed: " + e.getMessage());
        }
    }
}