package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableValue<T>(val initValue: T, val name: String, val checkValue: (T) -> T = {it}) {
    private val observable: Subject<DataWithLast<T>, DataWithLast<T>> = BehaviorSubject.create()
    val subscribers = ArrayList<Subscriber>()
    private var value = initValue

    var lastTotalUpdateTime = 0L

    init {
        observable.onNext(DataWithLast(initValue, initValue))
        observable.subscribe { value = it.newValue }

        ObservableContext.observableValues.add(this)
    }

    fun subscribe(subject: Any, description: String, onNext: (T) -> Unit): Subscriber {
        val subscription = observable.subscribe { dataWithLast ->
            onNext(dataWithLast.newValue)
        }
        val subscriber = Subscriber(subject, subscription, description)
        subscribers.add(subscriber)
        return subscriber
    }

    fun subscribeWithLast(subject: Any, description: String, onNext: (T, T) -> Unit): Subscriber {
        val subscription = observable.subscribe { dataWithLast ->
            onNext(dataWithLast.newValue, dataWithLast.oldValue)
        }
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
        val startTime = System.nanoTime()
        observable.onNext(DataWithLast(checkValue(action(value)), value))
        lastTotalUpdateTime = System.nanoTime() - startTime
    }

    fun update(newValue: T) {
        val startTime = System.nanoTime()
        observable.onNext(DataWithLast(checkValue(newValue), value))
        lastTotalUpdateTime = System.nanoTime() - startTime
    }

    fun get() = value

    fun once(onNext: (T) -> Unit) {
        observable.subscribe { dataWithLast ->
            onNext(dataWithLast.newValue)
        }.unsubscribe()
    }

    fun reset() {
        update(initValue)
    }

    data class Subscriber(val subject: Any, val subscription: Subscription, val description: String)

    data class DataWithLast<T>(val newValue: T, val oldValue: T)
}