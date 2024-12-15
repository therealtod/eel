import eelst.ilike.common.InputReader
import eelst.ilike.engine.player.PlayerPOV

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
