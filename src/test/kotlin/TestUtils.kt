import eelst.ilike.common.InputReader
import eelst.ilike.engine.player.GameFromPlayerPOV

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): GameFromPlayerPOV {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
