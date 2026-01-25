import { useDeleteSession } from '@/api/generated/sessions/sessions';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { useQueryClient } from '@tanstack/react-query';

export interface SessionToDelete {
  id: string;
  trackName: string;
  carName: string;
  lapCount?: number;
}

interface DeleteSessionDialogProps {
  session: SessionToDelete | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDeleted?: () => void;
}

export function DeleteSessionDialog({
  session,
  open,
  onOpenChange,
  onDeleted,
}: DeleteSessionDialogProps) {
  const queryClient = useQueryClient();
  const deleteMutation = useDeleteSession();

  const handleDeleteConfirm = async () => {
    if (!session) return;

    try {
      await deleteMutation.mutateAsync({ sessionId: session.id });
      queryClient.invalidateQueries({ queryKey: ['/api/v1/sessions'] });
      onOpenChange(false);
      onDeleted?.();
    } catch (err) {
      console.error('Failed to delete session:', err);
    }
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Session?</AlertDialogTitle>
          <AlertDialogDescription>
            This will permanently delete the session at{' '}
            <span className="font-medium">{session?.trackName}</span> with{' '}
            <span className="font-medium">{session?.carName}</span>
            {session?.lapCount !== undefined && (
              <>
                {' '}
                and all {session.lapCount} lap{session.lapCount !== 1 ? 's' : ''}
              </>
            )}{' '}
            with their telemetry data. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={handleDeleteConfirm}
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete Session'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
