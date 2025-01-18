package engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.knowledge.BaseSlotKnowledge
import testcommon.CommonData
import eelst.ilike.engine.knowledge.HandKnowledgeImpl
import eelst.ilike.engine.knowledge.TeamKnowledgeImpl
import eelst.ilike.game.GloballyAvailableGameDataFactory
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.suit.Suit
import game.entity.suit.Blue
import game.entity.suit.Red
import game.entity.variant.NoVariant
import io.mockk.clearAllMocks
import io.mockk.mockk
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class TeamKnowledgeImplTest {
    private val allNoVariantCards = NoVariant.getSuits().flatMap { it.getAllUniqueSuitCards() }.toSet()
    private val globallyAvailableGameData = GloballyAvailableGameDataFactory.createGloballyAvailableGameData(
        variant = NoVariant,
        playersMetadata = listOf(
            CommonData.aliceMetadata,
            CommonData.bobMetadata,
            CommonData.cathyMetadata,
        )
    )
    private val emptyTeamKnowledge = TeamKnowledgeImpl(
        slotsKnowledge = List(3) { emptyList() },
        handsKnowledge = List(3) { HandKnowledgeImpl() },
    )
    private val someTeamKnowledge = TeamKnowledgeImpl(
        slotsKnowledge = listOf(
            listOf(
                BaseSlotKnowledge(
                    slotOwnerPlayerIndex = 0,
                    empathyPerPlayer = listOf(
                        allNoVariantCards,
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.TWO,
                            )
                        ),
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.TWO,
                            )
                        ),
                    ),
                ),
            ),
            listOf(
                BaseSlotKnowledge(
                    slotOwnerPlayerIndex = 0,
                    empathyPerPlayer = listOf(
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.FIVE,
                            )
                        ),
                        allNoVariantCards,
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.FIVE,
                            )
                        ),
                    )
                ),
            ),
            listOf(
                BaseSlotKnowledge(
                    slotOwnerPlayerIndex = 0,
                    empathyPerPlayer = listOf(
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.TWO,
                            )
                        ),
                        setOf(
                            HanabiCard(
                                suit = Red,
                                Rank.TWO,
                            )
                        ),
                        allNoVariantCards,
                    ),
                ),
            )
        ),
        handsKnowledge = List(3) { HandKnowledgeImpl() }
    )
    private val conventionSet = mockk<ConventionSet>()

    @BeforeEach
    fun beforeEach() {
        clearAllMocks()
    }

    @Test
    fun `Should create a new slot knowledge with empathy equal to all possible cards When there is no previous knowledge`() {
        val drawAction = DrawAction(
            playerMetadata = CommonData.aliceMetadata
        )
        val updatedKnowledge = emptyTeamKnowledge.getAfterDraw(
            drawAction,
            globallyAvailableGameData,
        )
        val newSlotEmpathy = updatedKnowledge.getSlotKnowledge(0, 1)

        val expected = allNoVariantCards
        val actual = newSlotEmpathy.getEmpathy(0)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should create a new slot knowledge with the correct empathy for Alice Given previous Knowledge`() {
        val drawAction = DrawAction(
            playerMetadata = CommonData.aliceMetadata
        )
        val updatedKnowledge = someTeamKnowledge.getAfterDraw(drawAction, globallyAvailableGameData)
        val newSlotEmpathy = updatedKnowledge.getSlotKnowledge(0, 1)

        val expected = allNoVariantCards.minus(HanabiCard(Red, Rank.FIVE))
        val actual = newSlotEmpathy.getEmpathy(0)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should create a new slot knowledge with the correct empathy for Bob Given previous Knowledge`() {
        val drawAction = DrawAction(
            playerMetadata = CommonData.aliceMetadata
        )
        val updatedKnowledge = someTeamKnowledge.getAfterDraw(drawAction, globallyAvailableGameData)
        val newSlotEmpathy = updatedKnowledge.getSlotKnowledge(0, 1)

        val expected = allNoVariantCards.minus(HanabiCard(Red, Rank.TWO))
        val actual = newSlotEmpathy.getEmpathy(1)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should update existing slot knowledge When a new visible card is drawn`() {
        val drawAction = DrawAction(
            playerMetadata = CommonData.aliceMetadata
        )
        val updatedKnowledge = someTeamKnowledge.getAfterDrawing(
            drawAction,
            HanabiCard(Blue, Rank.FIVE),
            globallyAvailableGameData,
        )
        val newSlotEmpathy = updatedKnowledge.getSlotKnowledge(1, 1)

        val expected = allNoVariantCards.minus(HanabiCard(Red, Rank.TWO)).minus(HanabiCard(Blue, Rank.FIVE))
        val actual = newSlotEmpathy.getEmpathy(1)

        Assertions.assertEquals(expected, actual)
    }

    @Disabled
    @Test
    fun `Should remove the slot knowledge connected to a slot When the slot gets played`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1
        )
        val updatedKnowledge = someTeamKnowledge.getAfterPlay(
            playAction,
            globallyAvailableGameData,
        )
        TODO()
    }
}
