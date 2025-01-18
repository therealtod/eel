package eelst.ilike.engine.strategy

import eelst.ilike.common.datastructure.tree.Tree
import eelst.ilike.common.datastructure.tree.TreeNode
import eelst.ilike.common.datastructure.tree.TreeNodeImpl
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

/**
 * Determine all possible game states select the best line based on the highest value leaf node
 */
class BruteForceActionSelectionStrategy(
    private val evaluator: GameStateEvaluator,
    private val maxDepth: Int,
) : ActionSelectionStrategy {
    override fun selectAction(gameState: GameState, conventionSet: ConventionSet): ConventionalAction {
        val rootNodeData = ActionSelectorSearchNodeData(
            action = null,
            gameState = gameState,
            evaluation = evaluator.evaluate(gameState)
        )
        val root = TreeNodeImpl(
            data = rootNodeData
        )
        val tree = Tree(root = root)
        buildTree(
            root = root,
            possibleActions = TODO(),
            gameState = gameState,
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
        gameState: GameState,
        depth: Int,
    ) {
        if (depth <= maxDepth) {
            possibleActions.forEach {
                val nextGameState = getUpdatedPOV(it, conventionSet, gameState)
                val evaluation = evaluator.evaluate(nextGameState)
                val nodeData = ActionSelectorSearchNodeData(
                    action = it,
                    gameState = gameState,
                    evaluation = evaluation
                )
                val child = TreeNodeImpl(data = nodeData)
                root.addChild(child)
                val newPossibleActions = TODO()
                buildTree(
                    root = child,
                    possibleActions = newPossibleActions,
                    gameState = nextGameState,
                    conventionSet = conventionSet,
                    depth = depth + 1,
                )
            }
        }
    }

    private fun getUpdatedPOV(
        conventionalAction: ConventionalAction,
        conventionSet: ConventionSet,
        gameState: GameState
    ): GameState {
        return when (val gameAction = conventionalAction.action) {
            is PlayAction -> {
                gameState.getAfter(
                    playAction = gameAction,
                    playedCard = TODO()
                )
            }

            is DiscardAction -> {
                gameState.getAfter(
                    discardAction = gameAction,
                    discardedCard = TODO()
                )
            }

            is ClueAction -> {
                gameState.getAfter(
                    clueAction = gameAction,
                    touchedSlotsIndexes = TODO()
                )
            }

            else -> throw IllegalStateException(
                "No conventional action should be connected to a action of type: ${gameAction::class.simpleName}"
            )
        }
    }
}
