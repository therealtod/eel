import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): ActivePlayerPOV {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
