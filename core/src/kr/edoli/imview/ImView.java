package kr.edoli.imview;

import com.badlogic.gdx.Game;

public class ImView extends Game {

	@Override
	public void create() {
		setScreen(new ImageScreen());
	}
}
