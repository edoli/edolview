package kr.edoli.imview.util;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Pixmap;
import org.opencv.core.Mat;

import javax.swing.*;
import java.awt.*;
import java.awt.color.ColorSpace;
import java.awt.datatransfer.DataFlavor;
import java.awt.datatransfer.Transferable;
import java.awt.datatransfer.UnsupportedFlavorException;
import java.awt.image.*;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.stream.IntStream;

/**
 * Created by daniel on 16. 2. 28.
 */
public class Clipboard {
    public static Image paste() {
        Transferable transferable = Toolkit.getDefaultToolkit().getSystemClipboard().getContents(null);
        if (transferable != null && transferable.isDataFlavorSupported(DataFlavor.imageFlavor)) {
            try {
                return (Image) transferable.getTransferData(DataFlavor.imageFlavor);
            }
            catch (UnsupportedFlavorException e) {
                // handle this as desired
                e.printStackTrace();
            }
            catch (IOException e) {
                // handle this as desired
                e.printStackTrace();
            }
        }
        else {
            System.err.println("getImageFromClipboard: That wasn't an image!");
        }
        return null;
    }

    public static void copy(Mat mat) {
        copy(mat, 0, 0, mat.cols(), mat.rows());
    }

    public static void copy(Mat mat, int offsetX, int offsetY, int width, int height) {
        if (width == 0 || height == 0 ||
                offsetX < 0 || offsetY < 0 ||
                offsetX + width > mat.cols() || offsetY + height > mat.rows()) {
            return;
        }

        byte[] buff = new byte[(int) (mat.total() * mat.channels())];
        mat.get(0, 0, buff);

        int numChannels = mat.channels();
        boolean hasAlpha = (numChannels == 4);

        int copyNumChannels = numChannels;
        if (!hasAlpha) {
            copyNumChannels += 1;
        }

        int subLength = (width * height * copyNumChannels);
        byte[] subArray = new byte[subLength];

        for (int i = 0; i < subArray.length; i += copyNumChannels) {
            int index = i / copyNumChannels;
            int row = index / width;
            int col = index %  width;

            int j = (col + offsetX) * numChannels
                    + (row + offsetY) * numChannels * mat.cols();

            for (int k = 0; k < copyNumChannels; k++) {
                if (k < numChannels) {
                    subArray[i + k] = buff[j + k];
                } else {
                    subArray[i + k] = (byte) 255;
                }
            }
        }


        DataBufferByte data = new DataBufferByte(subArray, subLength);

        //the Raster
        int[] bandOffsets = IntStream.range(0, copyNumChannels).toArray();
        WritableRaster raster = Raster.createInterleavedRaster(data, width, height, width * copyNumChannels, copyNumChannels, bandOffsets, null);

        //the ColorModel
        int[] colorModels = {-1, -1, ColorSpace.CS_GRAY, -1, ColorSpace.CS_sRGB};
        ColorSpace cs = ColorSpace.getInstance(colorModels[copyNumChannels]);
        int[] nBits = IntStream.range(0, numChannels + 1).map(operand -> 8).toArray();
        ColorModel colorModel = new ComponentColorModel(cs, nBits, true, false,
                Transparency.TRANSLUCENT,
                DataBuffer.TYPE_BYTE);

        //the BufferedImage
        Image image = new BufferedImage(colorModel, raster, false, null);


        copy(image);

        //showImage(image);
    }

    public static void copy(String text) {
        Gdx.app.getClipboard().setContents(text);
    }

    public static void copy(Image image) {
        ImageSelection imgSel = new ImageSelection(image);
        Toolkit.getDefaultToolkit().getSystemClipboard().setContents(imgSel, null);
    }

    public static void showImage(Image image) {
        JLabel lblimage = new JLabel(new ImageIcon(image));

        JPanel mainPanel = new JPanel(new BorderLayout());
        mainPanel.add(lblimage);

        JFrame frame = new JFrame();
        frame.add(mainPanel);
        frame.setVisible(true);
        frame.setSize(300, 400);
    }

    public static class ImageSelection implements Transferable {
        private Image image;

        public ImageSelection(Image image) {
            this.image = image;
        }

        // Returns supported flavors
        public DataFlavor[] getTransferDataFlavors() {
            return new DataFlavor[] {
                DataFlavor.imageFlavor
            };
        }

        // Returns true if flavor is supported
        public boolean isDataFlavorSupported(DataFlavor flavor) {
            return DataFlavor.imageFlavor.equals(flavor);
        }

        // Returns image
        public Object getTransferData(DataFlavor flavor) throws UnsupportedFlavorException, IOException {
            if (!DataFlavor.imageFlavor.equals(flavor)) {
                throw new UnsupportedFlavorException(flavor);
            }
            return image;
        }
    }


}
