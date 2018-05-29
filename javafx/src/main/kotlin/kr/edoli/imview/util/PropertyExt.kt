package kr.edoli.imview.util

import javafx.beans.property.Property

fun Property<out Number>.bindDouble(observableValue: ObservableValue<Double>) {
    observableValue.subscribe(this) {
        value = it
    }
    addListener { _, _, newValue ->
        observableValue.update(newValue.toDouble())
    }
}

fun Property<Boolean>.bindBoolean(observableValue: ObservableValue<Boolean>) {
    observableValue.subscribe(this) {
        value = it
    }
    addListener { _, _, newValue ->
        observableValue.update(newValue)
    }
}

fun Property<String>.bindString(observableValue: ObservableValue<String>) {
    observableValue.subscribe(this) {
        value = it
    }
    addListener { _, _, newValue ->
        observableValue.update(newValue)
    }
}

fun Property<String>.bindOneWayString(observableValue: ObservableValue<*>, format: String? = null) {
    observableValue.subscribe(this) {
        value = format?.format(it) ?: it.toString()
    }
}