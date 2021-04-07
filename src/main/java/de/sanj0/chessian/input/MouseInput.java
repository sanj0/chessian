package de.sanj0.chessian.input;

import de.edgelord.saltyengine.core.Game;
import de.edgelord.saltyengine.displaymanager.stage.Stage;
import de.edgelord.saltyengine.input.MouseInputAdapter;
import de.edgelord.saltyengine.transform.Vector2f;
import de.edgelord.saltyengine.utils.GeneralUtil;
import de.sanj0.chessian.ChessScene;
import de.sanj0.chessian.ai.Chessian;
import de.sanj0.chessian.Pieces;
import de.sanj0.chessian.PlayerMoveState;
import de.sanj0.chessian.move.Move;
import de.sanj0.chessian.move.MoveGenerator;
import de.sanj0.chessian.utils.ChessianUtils;

import java.awt.event.MouseEvent;
import java.util.ArrayList;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

public class MouseInput extends MouseInputAdapter {

    private final ChessScene owner;
    private final Stage stage = Game.getHostAsDisplayManager().getStage();

    public MouseInput(final ChessScene owner) {
        this.owner = owner;
    }

    @Override
    public void mousePressed(final MouseEvent e) {
        final Vector2f cursor = cursorPosition(e);
        final PlayerMoveState playerMoveState = owner.getBoardRenderer().getMoveState();
        final int piece = PlayerMoveState.hoveredSquare(cursor);

        if (Pieces.color(owner.getBoard().get(piece)) == playerMoveState.getColorToMove()) {
            playerMoveState.setDraggedPieceIndex(piece);
            // replace with legal move generation
            playerMoveState.setLegalMoves(MoveGenerator.generateLegalMoves(piece, owner.getBoard()));
        }
    }

    @Override
    public void mouseReleased(final MouseEvent e) {
        final Vector2f cursor = cursorPosition(e);
        final PlayerMoveState playerMoveState = owner.getBoardRenderer().getMoveState();
        final int destination = PlayerMoveState.hoveredSquare(cursor);
        Move m;
        if (playerMoveState.getDraggedPieceIndex() != destination &&
                (m = ChessianUtils.getMoveByDestination(playerMoveState.getLegalMoves(), destination)) != null) {
            // do the move!
            // for now, simply replace piece values
            owner.getBoard().doMove(m);
            playerMoveState.nextTurn();
            if (owner.isAutoMove()) {
                CompletableFuture.runAsync(() -> {
                    Move response = null;
                    try {
                        response = Chessian.bestMove(owner.getBoard(), playerMoveState.getColorToMove());
                    } catch (ExecutionException | InterruptedException executionException) {
                        executionException.printStackTrace();
                    }
                    owner.getBoard().doMove(response);
                    playerMoveState.nextTurn();
                });
            }
        }

        owner.getBoardRenderer().getMoveState().setDraggedPieceIndex(-1);
        owner.getBoardRenderer().getMoveState().setLegalMoves(new ArrayList<>());
    }

    private Vector2f cursorPosition(final MouseEvent e) {
        final Vector2f cursorPos = new Vector2f(e.getX(), e.getY());
        final Vector2f imagePos = stage.getImagePosition();
        final float currentScale = stage.getCurrentScale();
        cursorPos.subtract(imagePos.getX(), imagePos.getY());
        cursorPos.divide(currentScale, currentScale);
        cursorPos.setX(GeneralUtil.clamp(cursorPos.getX(), 0, Game.getGameWidth()));
        cursorPos.setY(GeneralUtil.clamp(cursorPos.getY(), 0, Game.getGameHeight()));

        return cursorPos;
    }
}
