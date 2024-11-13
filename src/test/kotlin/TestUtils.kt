import eelst.ilike.engine.PlayerPOV
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return InputReader.parseFile("scenarios/scenario$scenarioId/pov.yaml")
    }
}
