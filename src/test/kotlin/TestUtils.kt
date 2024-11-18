import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
