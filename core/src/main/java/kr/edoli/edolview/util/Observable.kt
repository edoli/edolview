package kr.edoli.edolview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class Observable<T>(val name: String) {
    private val observable: Subject<T, T> = BehaviorSubject.create()
    val subscribers = ArrayList<Subscriber>()

    var lastTotalUpdateTime = 0L

    init {
        ObservableContext.observables.add(this)
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

    fun update(event: T) {
        if (!ObservableContext.push(this)) {
            return
        }

        val startTime = System.nanoTime()
        observable.onNext(event)
        lastTotalUpdateTime = System.nanoTime() - startTime

        ObservableContext.pop(this)
    }

    fun once(onNext: (T) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }

    data class Subscriber(val subject: Any, val subscription: Subscription, val description: String)
}