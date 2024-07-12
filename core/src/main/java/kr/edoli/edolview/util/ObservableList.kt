package kr.edoli.edolview.util

import rx.subjects.BehaviorSubject
import rx.subjects.Subject

/**
 * Created by daniel on 16. 10. 2.
 */
class ObservableList<T>(
        private var list: List<T> = ArrayList(),
        private val initialIndex: Int = 0,
        val name: String = "") {

    private val observable: Subject<ObservableListItem<T>, ObservableListItem<T>> = BehaviorSubject.create()
    val subscribers = ArrayList<ObservableValue.Subscriber>()

    val items: List<T>
        get() = list.toList()

    val size: Int
        get() = list.size

    var currentIndex = initialIndex
        private set

    var lastTotalUpdateTime = 0L

    init {
        updateInternal {}

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
        updateInternal { currentIndex = index }
    }

    fun update(newList: List<T>, newInitIndex: Int = -1) {
        updateInternal {
            list = newList
            currentIndex = newInitIndex
        }
    }

    fun update(action: (List<T>) -> List<T>) {
        updateInternal { list = action(list) }
    }

    private fun updateInternal(action: () -> Unit) {
        if (!ObservableContext.push(this)) {
            return
        }

        action()
        if (list.isEmpty()) {
            currentIndex = -1
        } else if (currentIndex > list.size) {
            currentIndex = list.size - 1
        }

        if (list.isNotEmpty() && currentIndex == -1) {
            currentIndex = 0
        }

        val startTime = System.nanoTime()
        observable.onNext(ObservableListItem(list, list.getOrNull(currentIndex)))
        lastTotalUpdateTime = System.nanoTime() - startTime

        ObservableContext.pop(this)
    }

    fun changeOrder(sourceIndex: Int, targetIndex: Int) {
        updateInternal {
            if (sourceIndex != targetIndex
                && sourceIndex >= 0
                && sourceIndex < list.size
                && targetIndex >= 0
                && targetIndex < list.size)
            {
                val arrayList = list

                if (arrayList is ArrayList) {
                    val source = arrayList.removeAt(sourceIndex)
                    arrayList.add(targetIndex, source)

                    if (currentIndex == sourceIndex) {
                        currentIndex = targetIndex
                    }

                } else {
                    throw Error("This ObservableList does not has ArrayList")
                }
            }
        }
    }

    fun get() = list.getOrNull(currentIndex)

    operator fun get(index: Int) = list.getOrNull(index)

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