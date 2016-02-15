package kr.edoli.imview;

import com.badlogic.gdx.Gdx;
import net.engio.mbassy.bus.MBassador;
import net.engio.mbassy.bus.config.BusConfiguration;
import net.engio.mbassy.bus.config.Feature;
import net.engio.mbassy.bus.error.IPublicationErrorHandler;
import net.engio.mbassy.bus.error.PublicationError;

/**
 * Created by 석준 on 2016-02-06.
 */
public class Bus {
    private static MBassador mBassador = new MBassador(new BusConfiguration()
            .addFeature(Feature.SyncPubSub.Default()) // configure the synchronous message publication
            .addFeature(Feature.AsynchronousHandlerInvocation.Default()) // configure asynchronous invocation of handlers
            .addFeature(Feature.AsynchronousMessageDispatch.Default())
            .addPublicationErrorHandler(new IPublicationErrorHandler() {
                @Override
                public void handleError(PublicationError error) {
                    Gdx.app.error("Bus", error.toString());
                }
            }));

    static {
    }

    public static void subscribe(Object listener) {
        mBassador.subscribe(listener);
    }

    public static void publish(Object message) {
        mBassador.publish(message);
    }
}
