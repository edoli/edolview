package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class NullableObservableValue<T>(initValue: T?) {
    val observable: Subject<T, T> = BehaviorSubject.create<T>()
    private val subjects = HashMap<Any, Subscription>()
    var value = initValue

    init {
        observable.onNext(initValue)
        observable.subscribe { value = it }
    }

    fun subscribe(onNext: (T?) -> Unit): Subscription {
        return observable.subscribe(onNext)
    }

    fun subscribe(subject: Any, onNext: (T?) -> Unit): Subscription {
        if (subject in subjects) {
            throw Exception()
        }

        val subscription = observable.subscribe(onNext)
        if (subscription != null) {
            subjects[subject] = subscription
        }
        return subscription
    }

    fun unsubscribe(subject: Any) {
        subjects[subject]?.unsubscribe()
    }

    fun update(action: (T?) -> T) {
        observable.onNext(action(value))
    }

    fun update(value: T?) {
        observable.onNext(value)
    }

    fun get() = value

    fun once(onNext: (T?) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }
}