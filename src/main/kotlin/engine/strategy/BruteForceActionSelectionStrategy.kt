package eelst.ilike.engine.strategy

import eelst.ilike.common.model.tree.Tree
import eelst.ilike.common.model.tree.TreeNode
import eelst.ilike.common.model.tree.TreeNodeImpl
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.slot.UnknownIdentitySlot
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

class BruteForceActionSelectionStrategy(
    private val evaluator: GameStateEvaluator,
): ActionSelectionStrategy {
    override fun selectAction(playerPOV: GameFromPlayerPOV, conventionSet: ConventionSet): ConventionalAction {
        val rootNodeData = ActionSelectorSearchNodeData(
            action = null,
            pov = playerPOV,
            evaluation = evaluator.evaluate(playerPOV)
        )
        val root = TreeNodeImpl(
            data = rootNodeData
        )
        val tree = Tree(root = root)
        buildTree(
            root = root,
            possibleActions = playerPOV.getLegalActions(conventionSet),
            playerPOV = playerPOV,
            conventionSet = conventionSet,
            depth = 0,
        )
        val bestLeafNode = tree.filter { it.isLeaf() }.maxBy { it.data.evaluation }
        val bestBranch = tree.getBranchToLeaf(bestLeafNode)

        return bestBranch[1].data.action!!
    }

    private fun buildTree(
        root: TreeNode<ActionSelectorSearchNodeData>,
        possibleActions: Collection<ConventionalAction>,
        conventionSet: ConventionSet,
        playerPOV: GameFromPlayerPOV,
        depth: Int,
    ) {
        if(depth <= MAX_DEPTH) {
            possibleActions.forEach {
                val updatedPOV = getUpdatedPOV(it, conventionSet, playerPOV)
                val evaluation = evaluator.evaluate(updatedPOV)
                val nodeData = ActionSelectorSearchNodeData(
                    action = it,
                    pov = updatedPOV,
                    evaluation = evaluation
                )
                val child = TreeNodeImpl(data = nodeData)
                root.addChild(child)
                val newPossibleActions = updatedPOV.getLegalActions(conventionSet)
                buildTree(
                    root = child,
                    possibleActions = newPossibleActions,
                    playerPOV = updatedPOV,
                    conventionSet = conventionSet,
                    depth = depth + 1,
                )
            }
        }
    }

    private fun getUpdatedPOV(
        conventionalAction: ConventionalAction,
        conventionSet: ConventionSet,
        playerPOV: GameFromPlayerPOV
    ): GameFromPlayerPOV {
        return when(val gameAction = conventionalAction.action) {
            is PlayAction -> {
                playerPOV.getAfterHypotheticalPlay(
                    playAction = gameAction,
                    conventionSet = conventionSet,
                )
            }
            is DiscardAction -> {
                playerPOV.getAfterHypotheticalDiscard(
                    discardAction = gameAction,
                    conventionSet = conventionSet,
                )
            }
            is ClueAction -> {
                playerPOV.getAfterHypotheticalClue(
                    clueAction = gameAction,
                    conventionSet = conventionSet,
                )
            }
            else -> throw IllegalStateException(
                "No conventional action should be connected to a action of type: ${gameAction::class.simpleName}"
            )
        }
    }


    companion object {
        private const val MAX_DEPTH = 5
    }
}