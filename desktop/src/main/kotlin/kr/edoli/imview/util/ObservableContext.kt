package kr.edoli.imview.util

object ObservableContext {
    val observableValues = mutableListOf<ObservableValue<*>>()
    val observableLists = mutableListOf<ObservableList<*>>()
}