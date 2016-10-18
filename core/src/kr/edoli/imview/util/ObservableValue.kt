package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableValue<T>(initValue: T) {
    private val observable: Subject<T, T> = BehaviorSubject.create<T>()
    private var value = initValue

    init {
        observable.onNext(initValue)
        observable.subscribe { value = it }
    }

    fun subscribe(onNext: (T) -> Unit): Subscription {
        return observable.subscribe(onNext)
    }

    fun update(action: (T) -> T) {
        value = action(value)
        observable.onNext(value)
    }

    fun update(value: T) {
        observable.onNext(value)
    }

    fun get() = value

    fun once(onNext: (T) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }
}