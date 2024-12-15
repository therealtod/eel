import eelst.ilike.common.InputReader2
import eelst.ilike.engine.player.PlayerPOV

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return InputReader2.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
