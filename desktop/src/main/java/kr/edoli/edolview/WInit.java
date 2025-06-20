package kr.edoli.edolview;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Window;
import org.lwjgl.system.*;

public final class WInit {
    public static void winit(Lwjgl3Window window) {

        if (Platform.get() == Platform.WINDOWS) {
            Windows.winit(window);
        }
    }
}