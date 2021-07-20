package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableLazyValue<T>(val initValue: T, val name: String, val checkValue: (T) -> T = {it}) {
    private val observable: Subject<T, T> = BehaviorSubject.create()
    val subscribers = ArrayList<Subscriber>()
    private var storedAction: (() -> T)? = null

    var lastTotalUpdateTime = 0L

    init {
        observable.onNext(initValue)
        ObservableContext.observableLazyValues.add(this)
    }

    fun subscribe(subject: Any, description: String, onNext: (T) -> Unit): Subscriber {
        executeUpdate()
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

    fun update(action: () -> T) {
        storedAction = action

        if (subscribers.size > 0) {
            executeUpdate()
        }
    }

    private fun executeUpdate() {
        val action = storedAction
        if (action != null) {
            val startTime = System.nanoTime()
            observable.onNext(checkValue(action()))
            lastTotalUpdateTime = System.nanoTime() - startTime
        }
        storedAction = null
    }

    fun once(onNext: (T) -> Unit) {
        executeUpdate()
        observable.subscribe(onNext).unsubscribe()
    }

    fun reset() {
        storedAction = null
        observable.onNext(initValue)
    }

    data class Subscriber(val subject: Any, val subscription: Subscription, val description: String)
}