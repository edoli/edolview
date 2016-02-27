package kr.edoli.imview.util;

/**
 * Created by daniel on 16. 2. 27.
 */
public class Histogram {
    private final int[] freq;
    private int max;

    // Create a new histogram.
    public Histogram(int n) {
        freq = new int[n];
    }

    // Add one occurrence of the value i.
    public void addDataPoint(int i) {
        freq[i]++;
        if (freq[i] > max) {
            max = freq[i];
        }
    }

    public int getNumber() {
        return freq.length;
    }

    public int getFreq(int i) {
        return freq[i];
    }

    public int getMaxFreq() {
        return max;
    }
}
