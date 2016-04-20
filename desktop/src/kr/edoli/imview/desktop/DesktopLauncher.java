package kr.edoli.imview.desktop;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration;
import com.badlogic.gdx.tools.texturepacker.TexturePacker;
import kr.edoli.imview.ImView;

public class DesktopLauncher {
	public static void main (String[] arg) {

		//pack();


		Lwjgl3ApplicationConfiguration config = new Lwjgl3ApplicationConfiguration();

        config.setWindowedMode(1280, 720);

		config.setBackBufferConfig(8, 8, 8, 8, 16, 0, 16);
		config.useVsync(false);

		if (arg.length > 0) {
			new Lwjgl3Application(new ImView(arg[0]), config);
		} else {
			new Lwjgl3Application(new ImView("test.png"), config);

		}

	}

	public static void pack() {
		TexturePacker.process("../../inputdir", ".", "images");
	}
}
