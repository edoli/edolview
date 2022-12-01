package kr.edoli.edolview.util

object ObservableContext {
    val observables = mutableListOf<Observable<*>>()
    val observableValues = mutableListOf<ObservableValue<*>>()
    val observableLazyValues = mutableListOf<ObservableLazyValue<*>>()
    val observableLists = mutableListOf<ObservableList<*>>()
}