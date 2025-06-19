package kr.edoli.edolview;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Window;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.system.*;

public final class WInit {
    public static void winit(Lwjgl3Window window) {
        long hwnd = GLFWNativeWin32.glfwGetWin32Window(window.getWindowHandle());
        if (hwnd == 0L) {
            System.err.println("Failed to get foreground HWND.");
            return;
        }

        // If OS is Windows, set the DWM attributes
        if (Platform.get() == Platform.WINDOWS) {
            Windows.winit(window);
        }
    }
}