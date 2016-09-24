package kr.edoli.imview.util;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Pixmap;

import javax.swing.*;
import java.awt.*;
import java.awt.color.ColorSpace;
import java.awt.datatransfer.DataFlavor;
import java.awt.datatransfer.Transferable;
import java.awt.datatransfer.UnsupportedFlavorException;
import java.awt.image.*;
import java.io.IOException;
import java.nio.ByteBuffer;

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

    public static void copy(Pixmap pixmap) {
        copy(pixmap, 0, 0, pixmap.getWidth(), pixmap.getHeight());
    }

    public static void copy(Pixmap pixmap, int offsetX, int offsetY, int width, int height) {
        if (width == 0 || height == 0 ||
                offsetX < 0 || offsetY < 0 ||
                offsetX + width > pixmap.getWidth() || offsetY + height > pixmap.getHeight()) {
            return;
        }

        ByteBuffer byteBuffer = pixmap.getPixels();

        int length = byteBuffer.capacity();
        byte[] totalArray = new byte[length];
        byteBuffer.get(totalArray, 0, length);
        byteBuffer.position(0);

        int subLength = (width * height * 4);
        byte[] subArray = new byte[subLength];

        for (int i = 0; i < subArray.length; i += 4) {
            int index = i / 4;
            int row = index / width;
            int col = index %  width;

            int j = (col + offsetX) * 4 + (row + offsetY) * 4 * pixmap.getWidth();
            subArray[i] = totalArray[j];
            subArray[i + 1] = totalArray[j + 1];
            subArray[i + 2] = totalArray[j + 2];
            subArray[i + 3] = totalArray[j + 3];
        }


        DataBufferByte data = new DataBufferByte(subArray, subLength);

        //the Raster
        int[] bandOffsets = new int[] {0, 1, 2, 3};
        WritableRaster raster = Raster.createInterleavedRaster(data, width, height, width * 4, 4, bandOffsets, null);

        //the ColorModel
        ColorSpace cs = ColorSpace.getInstance(ColorSpace.CS_sRGB);
        int[] nBits = {8, 8, 8, 8};
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
