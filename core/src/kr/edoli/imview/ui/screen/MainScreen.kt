package kr.edoli.imview.ui.screen

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.TextureRegion
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

        imageViewer.image = TextureRegion(Texture(Gdx.files.internal("test.png")))

        mainLayout.add(imageViewer).expand().fill()

        stage.addActor(mainLayout)
    }
}