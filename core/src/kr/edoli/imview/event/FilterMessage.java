package kr.edoli.imview.event;

import lombok.AllArgsConstructor;
import lombok.Data;

/**
 * Created by 석준 on 2016-02-06.
 */
@Data
@AllArgsConstructor
public class FilterMessage {
    public enum FilterType {
        DFilter, NoFilter
    }

    private FilterType type;
}
