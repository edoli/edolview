package kr.edoli.imview.util

import rx.subjects.PublishSubject
import rx.subjects.SerializedSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 9. 11.
 */
object Bus {
    val bus: Subject<Any, Any> = SerializedSubject<Any, Any>(PublishSubject.create())

    fun send(o: Any) {
        bus.onNext(o)
    }

    @Suppress("UNCHECKED_CAST")
    fun <T> subscribe(clazz: Class<T>, body: (T) -> Unit) {
        bus.subscribe {
            if (it.javaClass == clazz) {
                body(it as T)
            }
        }
    }
}