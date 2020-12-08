package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableValue<T>(private val initValue: T, val name: String = "") {
    private val observable: Subject<T, T> = BehaviorSubject.create<T>()
    val subjects = ArrayList<Subscriber>()
    private var value = initValue

    init {
        observable.onNext(initValue)
        observable.subscribe { value = it }

        ObservableContext.observableValues.add(this)
    }

    fun subscribe(onNext: (T) -> Unit): Subscription {
        return subscribe(null, onNext)
    }

    fun subscribe(subject: Any?, onNext: (T) -> Unit): Subscription {
        val subscription = observable.subscribe(onNext)
        if (subscription != null) {
            subjects.add(Subscriber(subject, subscription))
        }
        return subscription
    }

    fun unsubscribe(subject: Any) {
        subjects.removeIf { it.subject == subject }
    }

    fun update(action: (T) -> T) {
        value = action(value)
        observable.onNext(value)
    }

    fun update(newValue: T) {
        observable.onNext(newValue)
    }

    fun get() = value

    fun once(onNext: (T) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }

    fun reset() {
        update(initValue)
    }

    data class Subscriber(val subject: Any?, val subscription: Subscription)
}