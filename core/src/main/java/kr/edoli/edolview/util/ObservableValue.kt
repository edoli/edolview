package kr.edoli.edolview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableValue<T>(val initValue: T, val name: String, val checkValue: (T) -> T = {it}) {
    private val observable: Subject<T, T> = BehaviorSubject.create()
    val subscribers = ArrayList<Subscriber>()
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
        subscribers.add(subscriber)
        return subscriber
    }

    fun unsubscribe(subject: Any) {
        subscribers.filter { it.subject == subject }.forEach {
            unsubscribe(it)
        }
    }

    fun unsubscribe(subscriber: Subscriber) {
        subscriber.subscription.unsubscribe()
        subscribers.remove(subscriber)
    }

    fun update(action: (T) -> T) {
        if (!ObservableContext.push(this)) {
            return
        }

        value = checkValue(action(value))

        val startTime = System.nanoTime()
        observable.onNext(value)
        lastTotalUpdateTime = System.nanoTime() - startTime

        ObservableContext.pop(this)
    }

    fun update(newValue: T) {
        if (!ObservableContext.push(this)) {
            return
        }

        val startTime = System.nanoTime()
        observable.onNext(checkValue(newValue))
        lastTotalUpdateTime = System.nanoTime() - startTime

        ObservableContext.pop(this)
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