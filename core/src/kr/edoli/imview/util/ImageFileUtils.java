package kr.edoli.imview.util;

import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.ArrayUtils;

import java.io.File;
import java.util.Arrays;

/**
 * Created by daniel on 16. 2. 27.
 */
public class ImageFileUtils {
    public static File siblingFile(String currentFilePath, int modifier) {


        File file = new File(currentFilePath);

        if (file.getParentFile() == null) {
            return file;
        }

        String[] fileNames = file.getParentFile().list();
        String fileName = file.getName();
        Arrays.sort(fileNames);

        int index = ArrayUtils.indexOf(fileNames, fileName);
        String path;

        do {
            index = index + modifier;

            if (index == fileNames.length) {
                index = 0;
            }
            if (index == -1) {
                index = fileNames.length - 1;
            }

            path = file.getParent() + "/" + fileNames[index];
        } while (!isImage(path));

        return new File(path);
    }

    public static File nextFile(String currentFilePath) {
        return siblingFile(currentFilePath, 1);
    }

    public static File prevFile(String currentFilePath) {
        return siblingFile(currentFilePath, -1);
    }

    public static boolean isImage(String path) {
        String ext = FilenameUtils.getExtension(path);
        String[] exts = new String[] {"png", "PNG", "jpg", "JPG"};

        return ArrayUtils.contains(exts, ext);
    }
}
