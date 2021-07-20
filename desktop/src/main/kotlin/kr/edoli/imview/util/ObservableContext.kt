package kr.edoli.imview.util

object ObservableContext {
    val observableValues = mutableListOf<ObservableValue<*>>()
    val observableLazyValues = mutableListOf<ObservableLazyValue<*>>()
    val observableLists = mutableListOf<ObservableList<*>>()
}