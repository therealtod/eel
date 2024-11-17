import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.impl.ActivePlayer
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getActivePlayerFromScenario(scenarioId: Int): ActivePlayer{
        return InputReader.parseFile("scenarios/scenario$scenarioId/pov.yaml")
    }

    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return getActivePlayerFromScenario(scenarioId).playerPOV
    }
}
