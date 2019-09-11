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
    private val subjects = HashMap<Any, Subscription>()

    val items: List<T>
        get() = list.toList()

    var currentIndex = initialIndex
    var value = list[currentIndex]

    init {
        observable.onNext(list[initialIndex])
        observable.subscribe { value = it }
    }

    fun subscribe(onNext: (T) -> Unit): Subscription {
        return observable.subscribe(onNext)
    }

    fun subscribe(subject: Any, onNext: (T) -> Unit): Subscription {
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
        subjects.remove(subject)
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