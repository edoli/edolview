package kr.edoli.imview.util

import java.math.BigInteger
import java.util.regex.Pattern


class FilenameComparator : Comparator<String> {

    companion object {
        private val NUMBERS = Pattern.compile("(?<=\\D)(?=\\d)|(?<=\\d)(?=\\D)")
//        private val FILE_ENDING = Pattern.compile("(?<=.*)(?=\\..*)")
    }

    override fun compare(o1: String?, o2: String?): Int {
        if (o1 == null || o2 == null)
            return if (o1 == null) if (o2 == null) 0 else -1 else 1

        val split1 = NUMBERS.split(o1)
        val split2 = NUMBERS.split(o2)
        val length = Math.min(split1.size, split2.size)

        // Looping over the individual segments
        for (i in 0 until length) {
            val c1 = split1[i][0]
            val c2 = split2[i][0]
            var cmp = 0

            if (c1 in '0'..'9' && c2.code >= 0 && c2 <= '9') {
                cmp = BigInteger(split1[i]).compareTo(BigInteger(split2[i]))
            }

            if (cmp == 0) {
                cmp = split1[i].compareTo(split2[i])
            }

            if (cmp != 0) {
                return cmp
            }
        }

        var cmp = split1.size - split2.size
        return cmp

    }
}