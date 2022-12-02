package kr.edoli.edolview.util

lateinit var Platform: PlatformImp

enum class TouchState {
    None, Primary, Secondary
}

interface PlatformImp {
    fun showErrorMessage(message: String)
    fun getScalingFactor(): Float
    fun contentResolve(uri: String): ByteArray?
    fun getTouchState(): TouchState
}