package kr.edoli.edolview.util

import java.util.Stack

object ObservableContext {
    val observables = mutableListOf<Observable<*>>()
    val observableValues = mutableListOf<ObservableValue<*>>()
    val observableLazyValues = mutableListOf<ObservableLazyValue<*>>()
    val observableLists = mutableListOf<ObservableList<*>>()

    val observableStack = Stack<Any>()

    fun push(observable: Any): Boolean {
        if (observableStack.contains(observable)) {
            return false
        }
        observableStack.push(observable)
        return true
    }

    fun pop(observable: Any) {
        val popValue = observableStack.pop()
        if (popValue != observable) {
            throw Error("Observable stack does not match. Expected ${observable}, but get $popValue")
        }
    }
}