package kr.edoli.edolview.util

lateinit var Platform: PlatformImp

interface PlatformImp {
    fun showErrorMessage(message: String)
    fun getScalingFactor(): Float
    fun contentResolve(uri: String): ByteArray?
}