package kr.edoli.imview;

import lombok.AllArgsConstructor;
import lombok.Data;

/**
 * Created by 석준 on 2016-02-06.
 */
@Data
@AllArgsConstructor
public class FilterMessage {
    enum FilterType {
        DFilter, NoFilter
    }

    private FilterType type;
}
