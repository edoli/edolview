package kr.edoli.imview.bus

/**
 * Created by daniel on 16. 9. 24.
 */
class FileDropMessage(
        val windowName: String,
        val files: Array<String>
)