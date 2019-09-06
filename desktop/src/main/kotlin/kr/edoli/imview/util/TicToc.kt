package kr.edoli.imview.util

class TicTocTimer(val startTime: Long)

var globalTimer: TicTocTimer? = null

fun tic(): TicTocTimer {
    val timer = TicTocTimer(System.nanoTime())
    globalTimer = TicTocTimer(System.nanoTime())
    return timer
}

fun toc(tictocTimer: TicTocTimer? = null) {
    tictocTimer?.let {
        println(System.nanoTime() - it.startTime)
        return
    }
    globalTimer?.let {
        println(System.nanoTime() - it.startTime)
        return
    }
    System.err.println("tic() should be called before toc")
}