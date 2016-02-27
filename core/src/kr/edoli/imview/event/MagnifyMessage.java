package kr.edoli.imview.event;

import com.badlogic.gdx.math.Rectangle;
import lombok.AllArgsConstructor;
import lombok.Data;

/**
 * Created by 석준 on 2016-02-06.
 */
@Data
@AllArgsConstructor
public class MagnifyMessage {
    public MagnifyMessage(Rectangle rect) {
        this.x = rect.x;
        this.y = rect.y;
        this.width = rect.width;
        this.height = rect.height;
    }

    private float x;
    private float y;
    private float width;
    private float height;
}
