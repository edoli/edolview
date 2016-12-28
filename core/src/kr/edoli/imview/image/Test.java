package kr.edoli.imview.image;

import com.badlogic.gdx.Game;
import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration;
import com.badlogic.gdx.files.FileHandle;
import com.badlogic.gdx.graphics.Pixmap;
import org.opencv.core.Core;
import org.opencv.highgui.Highgui;
import org.opencv.imgproc.Imgproc;

import java.io.File;

/**
 * Created by sjjeon on 16. 12. 20.
 */
public class Test {
    public static void test() {
        System.out.println("!!!");
        Pixmap pixmap = new Pixmap(Gdx.files.internal("test.jpg"));

        for (int i = 0; i < 3; i++) {
            Pixmap tp = KotlinImageTest.INSTANCE.test(pixmap);
            Pixmap lp = JavaImageTest.test(pixmap);
        }

        Pixmap q = pixmap;
        for (int i = 0; i < 5; i++) {
            q = KotlinImageTest.INSTANCE.test(q);
        }

        long time1 = System.nanoTime();
        q = pixmap;
        for (int i = 0; i < 10; i++) {
            q = KotlinImageTest.INSTANCE.test(q);
        }
        long elapsed1 = (System.nanoTime() - time1) / 1000000;
        System.out.println(String.format("Kotlin: %dms", elapsed1));


        Pixmap p = pixmap;
        for (int i = 0; i < 5; i++) {
            p = JavaImageTest.test(p);
        }

        long time2 = System.nanoTime();
        p = pixmap;
        for (int i = 0; i < 10; i++) {
            p = JavaImageTest.test(p);
        }
        long elapsed2 = (System.nanoTime() - time2) / 1000000;
        System.out.println(String.format("Java: %dms", elapsed2));


    }
}
