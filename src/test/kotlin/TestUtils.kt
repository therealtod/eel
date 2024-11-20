import eelst.ilike.engine.player.OldActivePlayer
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.utils.InputReader

object TestUtils {
    fun getActivePlayerFromScenario(scenarioId: Int): OldActivePlayer {
        return InputReader.getPlayerFromResourceFile("scenarios/scenario$scenarioId/pov.yaml")
    }

    fun getPlayerPOVFromScenario(scenarioId: Int): PlayerPOV {
        return getActivePlayerFromScenario(scenarioId).playerPOV
    }
}
