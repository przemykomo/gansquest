package przemyk.gansquest;

import org.libsdl.app.SDLActivity;
import android.content.Intent;
import android.net.Uri;
import androidx.documentfile.provider.DocumentFile;

public class MyActivity extends SDLActivity {
    public static boolean pickDir = false;

    protected String[] getLibraries() {
        return new String[] { "SDL3", "gansquest" };
    }

    // gonna have to walk the directory in java and return all the file descriptors
    // through jni

    // Spaghetti based on https://github.com/libsdl-org/SDL/pull/9687
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (((MyActivity) mSingleton).pickDir == true) {
            ((MyActivity) mSingleton).pickDir = false;
            System.out.println("ABOBA ABOBA ABOBA ABOBA");
            System.out.println(data.getData().getPath()); // TODO: null check?

            Uri treeUri = data.getData();

            int flags = data.getFlags();

            int takeFlags =
                    flags & (Intent.FLAG_GRANT_READ_URI_PERMISSION
                           | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);

            // getContentResolver().takePersistableUriPermission(treeUri, takeFlags);

            getContentResolver().takePersistableUriPermission(
                    treeUri,
                    (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION)// & takeFlags
            );

            DocumentFile root = DocumentFile.fromTreeUri(this, treeUri);

            if (root != null && root.isDirectory()) {
                for (DocumentFile file : root.listFiles()) {
                    if (file.isDirectory()) {
                        System.out.println("TAG, Dir: " + file.getName());
                    } else {
                        System.out.println("TAG, File: " + file.getName());
                    }
                }
            }
        }
    }

    // waiting for https://github.com/libsdl-org/SDL/issues/9657
    public static void pickDirectory() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        mSingleton.startActivityForResult(intent, 0); // request code guess, SDL just increments it, so it might not
                                                      // work in general case
        ((MyActivity) mSingleton).pickDir = true;
        // startActivityForResult(intent, REQUEST_CODE_PICK_CUSTOM_DIRECTORY);
    }
}
