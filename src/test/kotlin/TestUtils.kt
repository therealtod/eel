import eelst.ilike.engine.Player
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getPlayerFromScenario(scenarioId: Int): Player {
        return InputReader.parseFile("scenarios/scenario$scenarioId/pov.yaml")
    }

    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return getPlayerFromScenario(scenarioId).playerPOV
    }
}
