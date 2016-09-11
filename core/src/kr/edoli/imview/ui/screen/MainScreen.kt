package kr.edoli.imview.ui.screen

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ui.view.ImageViewer

/**
 * Created by daniel on 16. 9. 10.
 */

class MainScreen : BaseScreen() {

    val imageViewer = ImageViewer()

    init {
        val mainLayout = Table()
        mainLayout.setFillParent(true)

        val imageViewWrapper = Table()
        imageViewer.image = TextureRegion(Texture(Gdx.files.internal("test.png")))
        imageViewWrapper.isTransform = true
        imageViewWrapper.clip = true
        imageViewWrapper.touchable = Touchable.childrenOnly
        imageViewWrapper.add(imageViewer).expand().fill()

        val centerLayout = Table()
        centerLayout.add(imageViewWrapper).expand().fill()

        mainLayout.add().height(32f).expandX().fillX().row()
        mainLayout.add(centerLayout).expand().fill().row()
        mainLayout.add().height(32f).expandX().fillX().row()

        mainLayout.debug()

        stage.addActor(mainLayout)
    }
}