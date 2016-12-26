package kr.edoli.imview.image;

import com.badlogic.gdx.files.FileHandle;
import com.badlogic.gdx.graphics.Pixmap;

import java.io.File;

/**
 * Created by sjjeon on 16. 12. 20.
 */
public class Test {
    public static void main(String[] args) {
        Pixmap pixmap = new Pixmap(new FileHandle(new File("wq")));

        for (int i = 0; i < 3; i++) {
            Pixmap tp = KotlinImageTest.INSTANCE.test(pixmap);
            Pixmap lp = JavaImageTest.test(pixmap);
        }

        long time1 = System.nanoTime();
        Pixmap q = pixmap;
        for (int i = 0; i < 10; i++) {
            q = KotlinImageTest.INSTANCE.test(q);
        }
        System.out.println((System.nanoTime() - time1) / 1000);

        long time2 = System.nanoTime();
        Pixmap p = pixmap;
        for (int i = 0; i < 10; i++) {
            p = KotlinImageTest.INSTANCE.test(p);
        }
        System.out.println((System.nanoTime() - time2) / 1000);
    }
}
