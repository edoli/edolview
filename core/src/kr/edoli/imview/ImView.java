package kr.edoli.imview;

import com.badlogic.gdx.Game;
import com.badlogic.gdx.Gdx;
import kr.edoli.imview.ui.MainScreen;
import lombok.Data;

public class ImView extends Game {
	private long targetDelay = 1000 / 60;
	private long start;

	public static AppArgs args = new AppArgs();

	public ImView(String imagePath) {
		args.setImagePath(imagePath);
	}

	@Override
	public void create() {
		Context.imagePath = ImView.args.getImagePath();

		setScreen(new MainScreen());
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

	@Data
	public static class AppArgs {
		private String imagePath;
	}
}
