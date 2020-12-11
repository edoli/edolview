package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableValue<T>(private val initValue: T, val name: String) {
    private val observable: Subject<T, T> = BehaviorSubject.create<T>()
    val subjects = ArrayList<Subscriber>()
    private var value = initValue

    var lastTotalUpdateTime = 0L

    init {
        observable.onNext(initValue)
        observable.subscribe { value = it }

        ObservableContext.observableValues.add(this)
    }

    fun subscribe(subject: Any, description: String, onNext: (T) -> Unit): Subscriber {
        val subscription = observable.subscribe(onNext)
        val subscriber = Subscriber(subject, subscription, description)
        subjects.add(subscriber)
        return subscriber
    }

    fun unsubscribe(subject: Any) {
        subjects.filter { it.subject == subject }.forEach {
            unsubscribe(it)
        }
    }

    fun unsubscribe(subscriber: Subscriber) {
        subscriber.subscription.unsubscribe()
        subjects.remove(subscriber)
    }

    fun update(action: (T) -> T) {
        value = action(value)

        val startTime = System.nanoTime()
        observable.onNext(value)
        lastTotalUpdateTime = System.nanoTime() - startTime
    }

    fun update(newValue: T) {
        val startTime = System.nanoTime()
        observable.onNext(newValue)
        lastTotalUpdateTime = System.nanoTime() - startTime
    }

    fun get() = value

    fun once(onNext: (T) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }

    fun reset() {
        update(initValue)
    }

    data class Subscriber(val subject: Any, val subscription: Subscription, val description: String)
}