package kr.edoli.edolview.res

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.ImageSpec
import java.io.File

abstract class Asset {
    companion object {

        private val urlProtocols = arrayOf("http", "https", "ftp")
        fun fromUri(uri: String): Asset {
            if (":" in uri) {
                val protocol = uri.split(":")[0]
                if (protocol in urlProtocols) {
                    return UrlAsset(uri)
                }
                // ex) 143.23.4.43:3432
                if (protocol.count { it == '.' } == 3) {
                    return UrlAsset(uri)
                }
                if (protocol == "content") {
                    return ContentAsset(uri)
                }
            }
            if ("/" in uri) {
                if (uri.split("/")[0].count { it == '.' } == 3) {
                    return UrlAsset(uri)
                }
            }

            return FileAsset(uri)
        }
    }

    abstract val name: String
    abstract val shouldAddToRecentAssets: Boolean

    open val workingDirectory: String = ""

    abstract fun retrieveImageSpec(): ImageSpec?
    abstract fun next(): Asset?
    abstract fun prev(): Asset?
    abstract fun checkRefresh(): Boolean
}