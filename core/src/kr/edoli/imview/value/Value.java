package kr.edoli.imview.value;

import com.badlogic.gdx.utils.Array;

/**
 * Created by daniel on 16. 2. 28.
 */
public class Value<T> {
    private T value;
    private Array<ValueListener<T>> listeners = new Array<>();

    public Value() {
    }

    public Value(T initialValue) {
        this.value = initialValue;
    }

    public T get() {
        return value;
    }

    public void set(T value) {
        this.value = value;

        for (ValueListener<T> listener : listeners) {
            listener.change(value);
        }
    }

    public void forceUpdate() {
        for (ValueListener<T> listener : listeners) {
            listener.change(value);
        }
    }

    public void addListener(ValueListener<T> listener) {
        listeners.add(listener);

        if (value != null) {
            listener.change(value);
        }
    }

    public void removeListener(ValueListener<T> listener) {
        listeners.removeValue(listener, true);
    }
}
