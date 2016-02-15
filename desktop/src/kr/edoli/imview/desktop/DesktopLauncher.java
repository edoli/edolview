package kr.edoli.imview.desktop;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration;
import kr.edoli.imview.ImView;

public class DesktopLauncher {
	public static void main (String[] arg) {
		Lwjgl3ApplicationConfiguration config = new Lwjgl3ApplicationConfiguration();

        config.setWindowedMode(1280, 720);

		config.setBackBufferConfig(8, 8, 8, 8, 16, 0, 16);
		config.useVsync(false);

		new Lwjgl3Application(new ImView(), config);
	}
}
