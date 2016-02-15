package kr.edoli.imview;

import com.badlogic.gdx.Game;
import com.badlogic.gdx.Gdx;

public class ImView extends Game {
	private long targetDelay = 1000 / 60;
	private long start;

	@Override
	public void create() {
		setScreen(new ImageScreen());
		start = System.currentTimeMillis();
	}



	@Override
	public void render() {
		long diff = System.currentTimeMillis() - start;

		long delay = targetDelay - diff;

		if (delay > 0) {
			try {
				Thread.sleep(delay);
			} catch (InterruptedException e) {
				e.printStackTrace();
			}
		}

		if (screen != null) screen.render(Gdx.graphics.getDeltaTime());

		start = System.currentTimeMillis();
	}
}
