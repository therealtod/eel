import eelst.ilike.common.InputReader
import eelst.ilike.engine.player.ActivePlayer

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): ActivePlayer {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
