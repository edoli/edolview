package kr.edoli.edolview.util

import rx.Subscription
import rx.subjects.PublishSubject

class Publisher<T>(val valueGenerator: () -> T) {


    private val subject = PublishSubject.create<T>()

    fun publish() {
        subject.onNext(valueGenerator())
    }

    fun subscribe(onNext: (T) -> Unit): Subscription {
        return subject.subscribe(onNext)
    }
}