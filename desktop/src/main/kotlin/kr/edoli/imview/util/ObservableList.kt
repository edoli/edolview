package kr.edoli.imview.util

import rx.Subscription
import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableList<T>(
        private var list: List<T>,
        private var initialIndex: Int = 0,
        val name: String = "") {

    private val observable: Subject<T, T> = BehaviorSubject.create<T>()
    val subjects = ArrayList<ObservableValue.Subscriber>()

    val items: List<T>
        get() = list.toList()

    var currentIndex = initialIndex
    var value = list[currentIndex]

    init {
        observable.onNext(list[initialIndex])
        observable.subscribe { value = it }

        ObservableContext.observableLists.add(this)
    }

    fun subscribe(subject: Any, description: String, onNext: (T) -> Unit): ObservableValue.Subscriber {
        val subscription = observable.subscribe(onNext)
        val subscriber = ObservableValue.Subscriber(subject, subscription, description)
        subjects.add(subscriber)
        return subscriber
    }

    fun unsubscribe(subject: Any) {
        subjects.filter { it.subject == subject }.forEach {
            unsubscribe(it)
        }
    }

    fun unsubscribe(subscriber: ObservableValue.Subscriber) {
        subscriber.subscription.unsubscribe()
        subjects.remove(subscriber)
    }

    fun update(index: Int) {
        currentIndex = index
        observable.onNext(list[index])
    }

    fun update(newList: List<T>, newInitIndex: Int = 0) {
        initialIndex = newInitIndex
        list = newList

        currentIndex = newInitIndex
        observable.onNext(newList[newInitIndex])
    }

    fun get() = value

    fun once(onNext: (T) -> Unit) {
        observable.subscribe(onNext).unsubscribe()
    }

    fun reset() {
        update(initialIndex)
    }
}