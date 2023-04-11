package kr.edoli.edolview.util

import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableList<T>(
        private var list: List<T>,
        private val initialIndex: Int = 0,
        val name: String = "") {

    private val observable: Subject<ObservableListItem<T>, ObservableListItem<T>> = BehaviorSubject.create()
    val subscribers = ArrayList<ObservableValue.Subscriber>()

    val items: List<T>
        get() = list.toList()

    val size: Int
        get() = list.size

    var currentIndex = initialIndex

    var lastTotalUpdateTime = 0L

    init {
        update()

        ObservableContext.observableLists.add(this)
    }

    fun subscribe(subject: Any,
                  description: String,
                  onNext: (list: List<T>, newValue: T?) -> Unit): ObservableValue.Subscriber {
        val subscription = observable.subscribe { onNext(it.list, it.newValue) }
        val subscriber = ObservableValue.Subscriber(subject, subscription, description)
        subscribers.add(subscriber)
        return subscriber
    }

    fun subscribeValue(subject: Any,
                  description: String,
                  onNext: (newValue: T?) -> Unit): ObservableValue.Subscriber {
        val subscription = observable.subscribe { onNext(it.newValue) }
        val subscriber = ObservableValue.Subscriber(subject, subscription, description)
        subscribers.add(subscriber)
        return subscriber
    }

    fun unsubscribe(subject: Any) {
        subscribers.filter { it.subject == subject }.forEach {
            unsubscribe(it)
        }
    }

    fun unsubscribe(subscriber: ObservableValue.Subscriber) {
        subscriber.subscription.unsubscribe()
        subscribers.remove(subscriber)
    }

    fun update(index: Int) {
        currentIndex = index

        update()
    }

    fun update(newList: List<T>, newInitIndex: Int = -1) {
        list = newList

        if (newInitIndex != -1) {
            currentIndex = newInitIndex
        }

        update()
    }

    fun update(action: (List<T>) -> List<T>) {
        list = action(list)

        update()
    }

    private fun update() {
        if (list.isEmpty()) {
            currentIndex = 0
        } else if (currentIndex > list.size) {
            currentIndex %= list.size
        }

        val startTime = System.nanoTime()
        observable.onNext(ObservableListItem(list, list.getOrNull(currentIndex)))
        lastTotalUpdateTime = System.nanoTime() - startTime
    }

    fun get() = list.getOrNull(currentIndex)

    fun once(onNext: (list: List<T>, currentValue: T?) -> Unit) {
        observable.subscribe {
            onNext(it.list, it.newValue)
        }.unsubscribe()
    }

    fun reset() {
        update(initialIndex)
    }

    fun release() {
        ObservableContext.observableLists.remove(this)
        subscribers.forEach { it.subscription.unsubscribe() }
        subscribers.clear()
    }

    data class ObservableListItem<T>(
        val list: List<T>,
        val newValue: T?,
    )
}